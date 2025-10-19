import Fuse from 'fuse.js';
import type { IFuseOptions } from 'fuse.js';
import type { WindowInfo } from '$lib/app/config';

export const fuseOptions: IFuseOptions<WindowInfo> = {
  keys: [
    { name: 'project', weight: 0.5 },
    { name: 'active_editor_tab', weight: 0.3 },
    { name: 'app_name', weight: 0.1 },
    { name: 'window_name', weight: 0.1 }
  ],
  threshold: 0.3,
  distance: 100,
  includeScore: true,
  includeMatches: true,
  minMatchCharLength: 1,
  shouldSort: true,
  findAllMatches: false,
  location: 0,
  ignoreLocation: false,
  ignoreFieldNorm: false
};

export function createFuseInstance(data: WindowInfo[]): Fuse<WindowInfo> {
  return new Fuse(data, fuseOptions);
} 