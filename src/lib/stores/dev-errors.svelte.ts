import { dev } from '$app/environment';

type DevError = {
  id: string;
  message: string;
  timestamp: number;
  data?: any;
};

class DevErrorsStore {
  errors = $state<DevError[]>([]);
  
  addError(message: string, data?: any) {
    if (!dev) return;
    
    const error: DevError = {
      id: `${Date.now()}-${Math.random()}`,
      message,
      timestamp: Date.now(),
      data
    };
    
    this.errors = [...this.errors, error];
    console.error('[DEV ERROR]', message, $state.snapshot(data));
  }
  
  clear() {
    this.errors = [];
  }
  
  get hasErrors() {
    return this.errors.length > 0;
  }
  
  get latestError() {
    return this.errors[this.errors.length - 1];
  }
}

export const devErrorsStore = new DevErrorsStore();
