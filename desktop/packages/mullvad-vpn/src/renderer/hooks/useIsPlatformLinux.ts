export const useIsPlatformLinux = () => {
  const isLinux = window.env.platform === 'linux';

  return isLinux;
};
