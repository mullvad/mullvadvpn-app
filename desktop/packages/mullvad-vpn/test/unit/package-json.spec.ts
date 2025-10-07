/**
 * Package.json Validation Tests
 * 
 * Tests for validating the structure and content of package.json files
 * after the Electron update from 36.5.0 to 37.6.0
 */

import { expect } from 'chai';
import { describe, it, before, after } from 'mocha';
import fs from 'fs/promises';
import path from 'path';

describe('Package.json Validation', () => {
  describe('Root package.json', () => {
    let packageJson: any;

    before(async () => {
      const packagePath = path.resolve(__dirname, '../../../../package.json');
      const content = await fs.readFile(packagePath, 'utf-8');
      packageJson = JSON.parse(content);
    });

    it('should be valid JSON', () => {
      expect(packageJson).to.not.be.undefined;
      expect(typeof packageJson).to.equal('object');
    });

    it('should have required top-level fields', () => {
      expect(packageJson).to.have.property('name');
      expect(packageJson).to.have.property('version');
      expect(packageJson).to.have.property('devDependencies');
    });

    it('should have updated @types/node to correct version', () => {
      expect(packageJson.devDependencies['@types/node']).to.equal('^22.18.8');
    });

    it('should have valid volta configuration', () => {
      expect(packageJson).to.have.property('volta');
      expect(packageJson.volta).to.have.property('node');
      expect(packageJson.volta).to.have.property('npm');
    });

    it('should have updated Node.js version in volta', () => {
      expect(packageJson.volta.node).to.equal('22.19.0');
    });

    it('should have updated npm version in volta', () => {
      expect(packageJson.volta.npm).to.equal('11.6.1');
    });

    it('should have valid semver versions', () => {
      const semverRegex = /^\d+\.\d+\.\d+$/;
      expect(packageJson.volta.node).to.match(semverRegex);
      expect(packageJson.volta.npm).to.match(semverRegex);
    });

    it('should have engines field', () => {
      expect(packageJson).to.have.property('engines');
      expect(packageJson.engines).to.have.property('node');
      expect(packageJson.engines).to.have.property('npm');
    });
  });

  describe('mullvad-vpn package.json', () => {
    let packageJson: any;

    before(async () => {
      const packagePath = path.resolve(__dirname, '../../package.json');
      const content = await fs.readFile(packagePath, 'utf-8');
      packageJson = JSON.parse(content);
    });

    it('should be valid JSON', () => {
      expect(packageJson).to.not.be.undefined;
      expect(typeof packageJson).to.equal('object');
    });

    it('should have updated Electron to 37.6.0', () => {
      expect(packageJson.devDependencies.electron).to.equal('37.6.0');
    });

    it('should have exact Electron version (not caret or tilde)', () => {
      const electronVersion = packageJson.devDependencies.electron;
      expect(electronVersion).to.not.contain('^');
      expect(electronVersion).to.not.contain('~');
      expect(electronVersion).to.match(/^\d+\.\d+\.\d+$/);
    });

    it('should have required devDependencies', () => {
      expect(packageJson.devDependencies).to.have.property('electron');
      expect(packageJson.devDependencies).to.have.property('electron-builder');
    });

    it('should have Electron version >= 37.0.0', () => {
      const version = packageJson.devDependencies.electron;
      const majorVersion = parseInt(version.split('.')[0], 10);
      expect(majorVersion).to.be.at.least(37);
    });
  });

  describe('Package.json integrity', () => {
    it('should have consistent dependency versions between root and mullvad-vpn', async () => {
      const rootPackagePath = path.resolve(__dirname, '../../../../package.json');
      const vpnPackagePath = path.resolve(__dirname, '../../package.json');
      
      const rootContent = await fs.readFile(rootPackagePath, 'utf-8');
      const vpnContent = await fs.readFile(vpnPackagePath, 'utf-8');
      
      const rootPackage = JSON.parse(rootContent);
      const vpnPackage = JSON.parse(vpnContent);

      // Check that shared dependencies use compatible versions
      const sharedDeps = ['eslint', 'prettier'];
      
      sharedDeps.forEach(dep => {
        if (rootPackage.devDependencies?.[dep] && vpnPackage.devDependencies?.[dep]) {
          // Versions should at least have the same major version
          const rootMajor = rootPackage.devDependencies[dep].match(/\d+/)?.[0];
          const vpnMajor = vpnPackage.devDependencies[dep].match(/\d+/)?.[0];
          expect(rootMajor).to.equal(vpnMajor);
        }
      });
    });

    it('should have Electron version that addresses macOS CPU issue', () => {
      // The update specifically fixes high CPU usage on macOS 15 (Sequoia)
      // This test documents that the version should be >= 37.6.0
      const packagePath = path.resolve(__dirname, '../../package.json');
      
      fs.readFile(packagePath, 'utf-8').then(content => {
        const packageJson = JSON.parse(content);
        const electronVersion = packageJson.devDependencies.electron;
        const [major, minor] = electronVersion.split('.').map(Number);
        
        const isValidVersion = major > 37 || (major === 37 && minor >= 6);
        expect(isValidVersion).to.be.true;
      });
    });
  });

  describe('Version format validation', () => {
    it('should use semantic versioning for Electron', async () => {
      const packagePath = path.resolve(__dirname, '../../package.json');
      const content = await fs.readFile(packagePath, 'utf-8');
      const packageJson = JSON.parse(content);
      
      const semverRegex = /^\d+\.\d+\.\d+$/;
      expect(packageJson.devDependencies.electron).to.match(semverRegex);
    });

    it('should use semantic versioning for Node.js in volta', async () => {
      const packagePath = path.resolve(__dirname, '../../../../package.json');
      const content = await fs.readFile(packagePath, 'utf-8');
      const packageJson = JSON.parse(content);
      
      const semverRegex = /^\d+\.\d+\.\d+$/;
      expect(packageJson.volta.node).to.match(semverRegex);
      expect(packageJson.volta.npm).to.match(semverRegex);
    });
  });
});

// Helper function tests
describe('Version comparison utilities', () => {
  function compareVersions(v1: string, v2: string): number {
    const parts1 = v1.split('.').map(Number);
    const parts2 = v2.split('.').map(Number);
    
    for (let i = 0; i < 3; i++) {
      if (parts1[i] > parts2[i]) return 1;
      if (parts1[i] < parts2[i]) return -1;
    }
    return 0;
  }

  it('should correctly compare version numbers', () => {
    expect(compareVersions('37.6.0', '36.5.0')).to.be.greaterThan(0);
    expect(compareVersions('36.5.0', '37.6.0')).to.be.lessThan(0);
    expect(compareVersions('37.6.0', '37.6.0')).to.equal(0);
  });

  it('should handle major version differences', () => {
    expect(compareVersions('38.0.0', '37.6.0')).to.be.greaterThan(0);
  });

  it('should handle minor version differences', () => {
    expect(compareVersions('37.7.0', '37.6.0')).to.be.greaterThan(0);
  });

  it('should handle patch version differences', () => {
    expect(compareVersions('37.6.1', '37.6.0')).to.be.greaterThan(0);
  });
});