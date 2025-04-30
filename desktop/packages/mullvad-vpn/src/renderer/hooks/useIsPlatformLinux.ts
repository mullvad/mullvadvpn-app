export const useIsPlatformLinux = () => {
  return false;
  const isPlatformLinux = window.env.platform === 'linux';

  return isPlatformLinux;
};
