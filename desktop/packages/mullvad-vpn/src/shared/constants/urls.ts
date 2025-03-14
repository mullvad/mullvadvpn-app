// This should only contain links to the mullvad website
// No links to other websites should be added
export const urls = {
  purchase: 'https://mullvad.net/account/',
  faq: 'https://mullvad.net/help/tag/mullvad-app/',
  privacyGuide: 'https://mullvad.net/help/first-steps-towards-online-privacy/',
  download: 'https://mullvad.net/download/vpn/',
  removingOpenVpnBlog: 'https://mullvad.net/en/blog/removing-openvpn-15th-january-2026',
} as const;

type BaseUrl = (typeof urls)[keyof typeof urls];
type ExtendedBaseUrl = `${BaseUrl}${string}`;
export type Url = BaseUrl | ExtendedBaseUrl;
