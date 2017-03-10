import Enum from './lib/enum';

/**
 * Login state enum used for React components
 * @type {enum}
 * @property {string} none        Initial state (not logged in)
 * @property {string} connecting  Attempting to log in
 * @property {string} failed      Failed to log in
 * @property {string} ok          Logged in
 */
export const LoginState = Enum('none', 'connecting', 'failed', 'ok');

/**
 * Connection state enum used for React components
 * @type {enum}
 * @property {string} disconnected  Initial state (disconnected)
 * @property {string} connecting    Connecting
 * @property {string} connected     Connected
 * @property {string} failed        Failed to connect
 */
export const ConnectionState = Enum('disconnected', 'connecting', 'connected', 'failed');
