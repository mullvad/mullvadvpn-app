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
 */
export const ConnectionState = new Enum('disconnected', 'connecting', 'connected');

/**
 * Tray icon type
 * @type {TrayIconType}
 * @property {string} unsecured - Initial state (unlocked)
 * @property {string} securing  - Securing network (spinner)
 * @property {string} secured   - Connection is secured (locked)
 */
export const TrayIconType = new Enum('unsecured', 'securing', 'secured');
