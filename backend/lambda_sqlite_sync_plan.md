# Lambda-SQLite Synchronization Protocol

This document outlines the state management and synchronization logic for maintaining a persistent SQLite database across ephemeral AWS Lambda execution environments using Amazon S3 as the backing store.

## 1. Architectural Constraints
*   **Persistence Layer:** Amazon S3 (Remote).
*   **Execution Layer:** AWS Lambda (Ephemeral `/tmp` storage).
*   **Concurrency Model:** Strict `Reserved Concurrency = 1` to ensure linearizability and prevent write-collision/data-loss.

## 2. Execution Lifecycle

### Phase 1: State Initialization & Validation (Ingress)
Upon invocation, the execution environment must ensure the local SQLite file in `/tmp` is consistent with the remote S3 HEAD.

1.  **Local Cache Check:** Verify existence of the SQLite binary and its associated metadata (ETag) in the `/tmp` directory.
2.  **Conditional Synchronization:**
    *   **Cold Start (No local cache):** Execute a `GET Object` request to S3. Persist the object to `/tmp` and store the `ETag` in memory.
    *   **Warm Start (Local cache exists):** Execute a `HEAD Object` or `GET Object` with an `If-None-Match` header using the cached `ETag`.
        *   **HTTP 304 (Not Modified):** Proceed using the existing local cache.
        *   **HTTP 200 (Modified):** Atomically replace the local cache with the new S3 payload and update the cached `ETag`.

### Phase 2: Transaction Execution
The application logic operates against the local SQLite instance.

*   **Read Operations:** Standard SQL execution. No state mutation occurs.
*   **Write Operations:** SQL `INSERT/UPDATE/DELETE`. Set an internal `is_dirty` boolean flag to `true` upon successful transaction commit to the local filesystem.

### Phase 3: State Persistence (Egress)
Before the Lambda function returns a response, the local state must be reconciled with the remote store if mutations occurred.

1.  **Mutation Check:** Evaluate the `is_dirty` flag.
2.  **State Commitment:**
    *   If `is_dirty` is `false`: Terminate the execution lifecycle.
    *   If `is_dirty` is `true`:
        1.  Flush all SQLite WAL/journal buffers to disk.
        2.  Execute a `PUT Object` request to S3 containing the updated SQLite file.
        3.  Update the in-memory `ETag` with the value returned in the S3 response.
        4.  Reset `is_dirty` to `false`.

## 3. Synchronization Logic Summary

| Condition | Action |
| :--- | :--- |
| **Null Local State** | Fetch full object from S3; Initialize metadata. |
| **Stale Local State** | Re-fetch object via ETag mismatch; Update cache. |
| **Read-Only Request** | No egress synchronization required. |
| **Mutated State** | Atomic S3 Upload (PUT) required before response. |

## 4. Concurrency & Integrity
To maintain data integrity and prevent "lost updates," the system relies on **Single Writer Semantics**. By enforcing `Reserved Concurrency = 1`, we guarantee that only one execution environment can possess a "Dirty" state at any given time, effectively serializing all write operations and ensuring the S3 object remains the authoritative, sequential source of truth.
