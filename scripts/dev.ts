import concurrently from 'concurrently';
import * as path from 'path';

const backendDir = path.resolve(process.cwd(), 'backend');
const frontendDir = path.resolve(process.cwd(), 'frontend');

const { result } = concurrently(
  [
    { 
      command: 'just watch', 
      name: 'backend', 
      cwd: backendDir, 
      prefixColor: 'blue' 
    },
    { 
      command: 'pnpm dev', 
      name: 'frontend', 
      cwd: frontendDir, 
      prefixColor: 'green' 
    }
  ],
  {
    prefix: 'name',
    killOthers: ['failure', 'success'],
    restartTries: 0,
  }
);

result.then(
  () => {
    console.log('Processes exited successfully.');
    process.exit(0);
  },
  (err) => {
    console.error('Processes exited with an error or were killed.', err);
    process.exit(1);
  }
);
