export const useIsPlatformLinux = () => {
  const isPlatformLinux = window.env.platform === 'linux';

  return isPlatformLinux;
};
