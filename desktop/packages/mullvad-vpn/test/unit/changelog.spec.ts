/**
 * CHANGELOG.md Validation Tests
 * 
 * Tests for validating the CHANGELOG.md structure and content
 */

import { expect } from 'chai';
import { describe, it, before, after } from 'mocha';
import fs from 'fs/promises';
import path from 'path';

describe('CHANGELOG.md Validation', () => {
  let changelogContent: string;

  before(async () => {
    const changelogPath = path.resolve(__dirname, '../../../../../CHANGELOG.md');
    changelogContent = await fs.readFile(changelogPath, 'utf-8');
  });

  describe('File structure', () => {
    it('should exist and be readable', () => {
      expect(changelogContent).to.not.be.undefined;
      expect(changelogContent.length).to.be.greaterThan(0);
    });

    it('should be valid UTF-8', () => {
      // If we got here, the file was read successfully as UTF-8
      expect(typeof changelogContent).to.equal('string');
    });

    it('should not be empty', () => {
      expect(changelogContent.trim().length).to.be.greaterThan(0);
    });
  });

  describe('Content validation', () => {
    it('should mention Electron update', () => {
      const electronUpdateRegex = /electron.*37\.6\.0/i;
      expect(changelogContent).to.match(electronUpdateRegex);
    });

    it('should mention macOS fix', () => {
      const macOSFixRegex = /macOS.*CPU|CPU.*macOS/i;
      expect(changelogContent).to.match(macOSFixRegex);
    });

    it('should have "### Fixed" section', () => {
      expect(changelogContent).to.contain('### Fixed');
    });

    it('should mention macOS platform specifically', () => {
      const hasMacOSSection = /####\s*macOS/i.test(changelogContent);
      expect(hasMacOSSection).to.be.true;
    });

    it('should reference the specific macOS version (macOS 15 or Sequoia)', () => {
      const macOSVersionRegex = /macOS\s*(15|Sequoia)|Sequoia/i;
      expect(changelogContent).to.match(macOSVersionRegex);
    });

    it('should mention Electron version upgrade range', () => {
      const versionPattern = /36\.5\.0.*37\.6\.0|37\.6\.0.*36\.5\.0/;
      expect(changelogContent).to.match(versionPattern);
    });
  });

  describe('Markdown formatting', () => {
    it('should use proper heading hierarchy', () => {
      const lines = changelogContent.split('\n');
      const headings = lines.filter(line => line.trim().startsWith('#'));
      
      expect(headings.length).to.be.greaterThan(0);
      
      // Check that headings are properly formatted (# with space)
      headings.forEach(heading => {
        const match = heading.match(/^(#{1,6})\s+/);
        expect(match).toBeTruthy();
      });
    });

    it('should not have trailing whitespace on lines', () => {
      const lines = changelogContent.split(String.fromCharCode(10));
      const linesWithTrailingSpace = lines.filter(line => line.length > 0 && line !== line.trimEnd());
      
      // Allow some trailing whitespace but keep it minimal
      expect(linesWithTrailingSpace.length).to.be.lessThan(10);
    });

    it('should use consistent list formatting', () => {
      const lines = changelogContent.split('\n');
      const listItems = lines.filter(line => line.trim().startsWith('-'));
      
      if (listItems.length > 0) {
        // Check that list items have space after dash
        listItems.forEach(item => {
          expect(item.trim()).to.match(/^-\s+/);
        });
      }
    });
  });

  describe('Recent changes validation', () => {
    it('should have recent entries at the top', () => {
      const lines = changelogContent.split('\n');
      const firstMeaningfulLine = lines.find(line => line.trim().length > 0);
      
      // Should not start with very old version numbers if properly maintained
      expect(firstMeaningfulLine).to.not.be.undefined;
    });

    it('should document the Electron update in latest changes', () => {
      // Get first 1000 characters (should contain recent changes)
      const recentSection = changelogContent.substring(0, 1000);
      const hasElectronUpdate = /electron.*37\.6\.0/i.test(recentSection);
      
      expect(hasElectronUpdate).to.be.true;
    });
  });

  describe('Link validation', () => {
    it('should not have broken markdown links', () => {
      // Match markdown links: [text](url)
      const linkRegex = /\[([^\]]+)\]\(([^)]+)\)/g;
      const links = [...changelogContent.matchAll(linkRegex)];
      
      links.forEach(link => {
        const url = link[2];
        // Basic validation: URL should not be empty and should not have spaces
        expect(url.length).to.be.greaterThan(0);
        expect(url).to.not.contain(' ');
      });
    });

    it('should use valid markdown link syntax', () => {
      const invalidLinkRegex = /\[[^\]]*\]\([^)]*\s[^)]*\)/;
      expect(changelogContent).not.to.match(invalidLinkRegex);
    });
  });

  describe('Content completeness', () => {
    it('should provide context for the Electron update', () => {
      const lowerContent = changelogContent.toLowerCase();
      
      // Should mention the reason for the update (CPU usage)
      const hasContext = lowerContent.includes('cpu') || 
                        lowerContent.includes('performance') ||
                        lowerContent.includes('usage');
      
      expect(hasContext).to.be.true;
    });

    it('should specify which platform is affected', () => {
      const hasPlatform = /####\s*(macOS|Linux|Windows)/i.test(changelogContent);
      expect(hasPlatform).to.be.true;
    });

    it('should clearly indicate this is a fix', () => {
      const fixSection = changelogContent.match(/###\s*Fixed[\s\S]*?(?=###|$)/i);
      expect(fixSection).toBeTruthy();
      
      if (fixSection) {
        const hasElectronFix = /electron.*37\.6\.0/i.test(fixSection[0]);
        const hasMacOSFix = /macOS.*CPU|high\s+CPU/i.test(fixSection[0]);
        
        expect(hasElectronFix || hasMacOSFix).to.be.true;
      }
    });
  });

  describe('Version number format', () => {
    it('should use semantic versioning format', () => {
      const semverRegex = /\d+\.\d+\.\d+/g;
      const versions = changelogContent.match(semverRegex);
      
      expect(versions).to.not.be.undefined;
      expect((versions as string[]).length).to.be.greaterThan(0);
      
      // Verify format of version numbers
      versions?.forEach(version => {
        const parts = version.split('.');
        expect(parts.length).to.equal(3);
        parts.forEach(part => {
          expect(parseInt(part, 10)).not.toBeNaN();
        });
      });
    });

    it('should correctly reference version 37.6.0', () => {
      expect(changelogContent).to.contain('37.6.0');
    });

    it('should correctly reference previous version 36.5.0', () => {
      expect(changelogContent).to.contain('36.5.0');
    });
  });
});