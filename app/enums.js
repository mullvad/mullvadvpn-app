import Enum from './lib/enum';

/**
 * Login state enum used for React components
 * @type {LoginState}
 * @property {string} none        Initial state (not logged in)
 * @property {string} connecting  Attempting to log in
 * @property {string} failed      Failed to log in
 * @property {string} ok          Logged in
 */
export const LoginState = new Enum('none', 'connecting', 'failed', 'ok');

/**
 * Connection state enum used for React components
 * @type {ConnectionState}
 * @property {string} disconnected  Initial state (disconnected)
 * @property {string} connecting    Connecting
 * @property {string} connected     Connected
 * @property {string} failed        Failed to connect
 */
export const ConnectionState = new Enum('disconnected', 'connecting', 'connected', 'failed');
