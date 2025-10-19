import { getCurrentWindow } from '@tauri-apps/api/window';
import { WINDOW_VISIBILITY_CHECK_INTERVAL } from '$lib/app/config';
import { logger } from '$lib/utils/logger';

class OverlayStore {
  isVisible = $state(false);
  onShowCallback = $state<(() => void) | null>(null);
  onHideCallback = $state<(() => void) | null>(null);
  
  private win = getCurrentWindow();
  private intervalId: number | null = null;

  constructor() {
    logger.info('OVERLAY_STORE', 'Initializing overlay store');
    logger.debug('OVERLAY_STORE', 'Window object created');
    this.startVisibilityCheck();
  }

  private async startVisibilityCheck() {
    logger.info('OVERLAY_STORE', `Starting visibility check with interval: ${WINDOW_VISIBILITY_CHECK_INTERVAL}ms`);
    
    const checkVisibility = async () => {
      try {
        const visible = await this.win.isVisible();
        const wasVisible = this.isVisible;
        
        if (visible !== wasVisible) {
          this.isVisible = visible;
          logger.info('OVERLAY_STORE', `Window visibility changed: ${wasVisible} -> ${visible}`);
          
          if (visible && this.onShowCallback) {
            logger.debug('OVERLAY_STORE', 'Triggering onShowCallback');
            this.onShowCallback();
          } else if (!visible && this.onHideCallback) {
            logger.debug('OVERLAY_STORE', 'Triggering onHideCallback');
            this.onHideCallback();
          }
        }
      } catch (error) {
        logger.error('OVERLAY_STORE', 'Error checking window visibility', error);
      }
    };

    await checkVisibility();
    const initialVisibility = this.isVisible;
    logger.info('OVERLAY_STORE', `Initial visibility: ${initialVisibility}`);
    
    this.intervalId = window.setInterval(checkVisibility, WINDOW_VISIBILITY_CHECK_INTERVAL);
    logger.info('OVERLAY_STORE', 'Visibility check interval started');
  }

  setOnShow(callback: () => void) {
    logger.debug('OVERLAY_STORE', 'Setting onShow callback');
    this.onShowCallback = callback;
  }

  setOnHide(callback: () => void) {
    logger.debug('OVERLAY_STORE', 'Setting onHide callback');
    this.onHideCallback = callback;
  }

  async show() {
    logger.info('OVERLAY_STORE', 'show() called');
    try {
      await this.win.show();
      logger.info('OVERLAY_STORE', 'Window.show() completed');
      await this.win.setFocus();
      logger.info('OVERLAY_STORE', 'Window.setFocus() completed');
    } catch (error) {
      logger.error('OVERLAY_STORE', 'Error in show()', error);
    }
  }

  async hide() {
    logger.info('OVERLAY_STORE', 'hide() called');
    try {
      await this.win.hide();
      logger.info('OVERLAY_STORE', 'Window.hide() completed');
    } catch (error) {
      logger.error('OVERLAY_STORE', 'Error in hide()', error);
    }
  }

  async toggle() {
    logger.info('OVERLAY_STORE', 'toggle() called');
    try {
      const visible = await this.win.isVisible();
      logger.info('OVERLAY_STORE', `Current visibility: ${visible ? 'visible' : 'hidden'}`);
      
      if (visible) {
        await this.hide();
      } else {
        await this.show();
      }
    } catch (error) {
      logger.error('OVERLAY_STORE', 'Error in toggle()', error);
    }
  }

  destroy() {
    if (this.intervalId !== null) {
      clearInterval(this.intervalId);
    }
  }
}

export const overlayStore = new OverlayStore();

