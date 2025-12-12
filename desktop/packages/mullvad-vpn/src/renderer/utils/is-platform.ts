type Platform = Extract<NodeJS.Platform, 'linux' | 'darwin' | 'win32'>;

export const isPlatform = (platform: Platform) => {
  const isPlatform = window.env.platform === platform;

  return isPlatform;
};
