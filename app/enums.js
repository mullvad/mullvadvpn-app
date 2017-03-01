import Enum from './lib/enum';

export const LoginState = Enum('none', 'connecting', 'failed', 'ok');
export const ConnectionState = Enum('disconnected', 'connecting', 'connected', 'failed');
