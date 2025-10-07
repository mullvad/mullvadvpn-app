import { expect } from 'chai';
import fs from 'fs';
import path from 'path';

describe('Package versions (electron bump, node types)', () => {
  it('packages/mullvad-vpn devDependencies.electron is 37.6.0', () => {
    const pkgPath = path.join(__dirname, '..', '..', '..', 'package.json');
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    expect(pkg.devDependencies.electron).to.equal('37.6.0');
  });

  it('desktop/package.json settings updated (volta.node, @types/node)', () => {
    const desktopPkgPath = path.join(__dirname, '..', '..', '..', '..', 'package.json');
    const pkg = JSON.parse(fs.readFileSync(desktopPkgPath, 'utf8'));
    expect(pkg.volta.node).to.equal('22.19.0');
    expect(pkg.devDependencies['@types/node']).to.equal('^22.18.8');
  });
});