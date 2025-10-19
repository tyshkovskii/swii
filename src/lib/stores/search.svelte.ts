import { createFuseInstance } from "$lib/fuse/config";
import type { WindowInfo } from "$lib/app/config";
import type Fuse from "fuse.js";

class SearchStore {
  query = $state("");

  clear() {
    this.query = "";
  }

  search(windows: WindowInfo[]) {
    const fuseInstance: Fuse<WindowInfo> = createFuseInstance(windows);

    if (!this.query.trim()) {
      return windows.map((window, index) => ({
        item: window,
        refIndex: index,
        score: 0,
      }));
    }

    return fuseInstance.search(this.query);
  }
}

export const searchStore = new SearchStore();
