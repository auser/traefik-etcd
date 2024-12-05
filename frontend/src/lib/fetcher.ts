export const baseUrl = import.meta.env.VITE_PROD ? '/api' : 'http://localhost:9090/api';

export const fetching = async (path: string, options: RequestInit = {}) => {
  return fetch(`${baseUrl}${path}`, options);
}