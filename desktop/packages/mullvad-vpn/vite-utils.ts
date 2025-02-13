// @see comments in vite.config.ts for an explanation of why this file exists.
import cp from 'node:child_process';

export interface PidTree {
  pid: number;
  ppid: number;
  children?: PidTree[];
}

export function treeKillSync(pid: number) {
  if (process.platform === 'win32') {
    cp.execSync(`taskkill /pid ${pid} /T /F`);
  } else {
    killTree(pidTree({ pid, ppid: pid }));
  }
}

function pidTree(tree: PidTree) {
  const command =
    process.platform === 'darwin'
      ? `pgrep -P ${tree.pid}` // Mac
      : `ps -o pid --no-headers --ppid ${tree.ppid}`; // Linux

  try {
    const childs = cp
      .execSync(command, { encoding: 'utf8' })
      .match(/\d+/g)
      ?.map((id) => +id);

    if (childs) {
      tree.children = childs.map((cid) => pidTree({ pid: cid, ppid: tree.pid }));
    }
  } catch {}

  return tree;
}

function killTree(tree: PidTree) {
  if (tree.children) {
    for (const child of tree.children) {
      killTree(child);
    }
  }

  try {
    process.kill(tree.pid); // #214
  } catch {
    /* empty */
  }
}
