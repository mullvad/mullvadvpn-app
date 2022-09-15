import { expect } from 'chai';
import { after, it, describe } from 'mocha';
import { parseChangelog } from '../../../src/main/changelog';

// It should be handled the same no matter if the platforms are split with a space or not.
const changelogItems = [
  'Changelog item 1',
  '[Windows] Changelog item 2',
  '[macOS] Changelog item 3',
  '[linux] Changelog item 4',
  '[Windows, macOS] Changelog item 5',
  '[Windows,linux] Changelog item 6',
  '[Windows, macOS,linux] Changelog item 7',
];

const changelogString = changelogItems.join('\n');

const mockPlatform = (platform: string) => {
  Object.defineProperty(process, 'platform', { value: platform });
};

describe('Changelog parser', () => {
  const platform = process.platform;

  after(() => {
    mockPlatform(platform);
  });

  it('should show Windows items', () => {
    mockPlatform('win32');

    const changelog = parseChangelog(changelogString);

    expect(changelog).to.have.length(5);
    expect(changelogItems[0].endsWith(changelog[0])).to.be.true;
    expect(changelogItems[1].endsWith(changelog[1])).to.be.true;
    expect(changelogItems[4].endsWith(changelog[2])).to.be.true;
    expect(changelogItems[5].endsWith(changelog[3])).to.be.true;
    expect(changelogItems[6].endsWith(changelog[4])).to.be.true;
  });

  it('should show macOS items', () => {
    mockPlatform('darwin');

    const changelog = parseChangelog(changelogString);

    expect(changelog).to.have.length(4);
    expect(changelogItems[0].endsWith(changelog[0])).to.be.true;
    expect(changelogItems[2].endsWith(changelog[1])).to.be.true;
    expect(changelogItems[4].endsWith(changelog[2])).to.be.true;
    expect(changelogItems[6].endsWith(changelog[3])).to.be.true;
  });

  it('should show Linux items', () => {
    mockPlatform('linux');

    const changelog = parseChangelog(changelogString);

    expect(changelog).to.have.length(4);
    expect(changelogItems[0].endsWith(changelog[0])).to.be.true;
    expect(changelogItems[3].endsWith(changelog[1])).to.be.true;
    expect(changelogItems[5].endsWith(changelog[2])).to.be.true;
    expect(changelogItems[6].endsWith(changelog[3])).to.be.true;
  });
});
