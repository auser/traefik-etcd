import { env } from '$env/dynamic/public';
import debugLib from 'debug';

const isProd = env.PUBLIC_PROD === 'true';

debugLib.enable(isProd ? 'app:info' : 'app:debug');
export const log = debugLib('app');
export const debug = log.extend('debug');
export const info = log.extend('info');
export const warn = log.extend('warn');
export const error = log.extend('error');