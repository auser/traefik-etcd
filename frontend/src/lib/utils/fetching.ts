import { env } from "$env/dynamic/public";
import { debug } from "./logging";
const API_BASE_URL = env.PUBLIC_API_BASE_URL || 'http://localhost:9090';
const isProd = env.PUBLIC_PROD === 'true';

export const baseUrl = isProd ? '/api' : `${API_BASE_URL}/api`;

export const fetching = async (path: string, options: RequestInit = {}) => {
  debug('fetching', `${baseUrl}${path}`);
  return fetch(`${baseUrl}${path}`, options);
}