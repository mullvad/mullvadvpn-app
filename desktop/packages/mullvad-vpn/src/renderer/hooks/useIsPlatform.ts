type Platform = Extract<NodeJS.Platform, 'linux' | 'darwin' | 'win32'>;

export const useIsPlatform = (platform: Platform) => {
  const isPlatform = window.env.platform === platform;

  return isPlatform;
};
