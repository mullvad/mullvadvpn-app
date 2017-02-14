import { expect } from 'chai';
import Enum from '../../app/lib/enum';

describe('enum', () => {
  
  it('should be able to compare values', () => {
    const e = Enum('NORTH', 'SOUTH', 'WEST', 'EAST');
    expect(e.NORTH).to.be.equal('NORTH');
  });

  it('should not be able to modify enum', () => {
    let e = Enum('NORTH', 'SOUTH', 'WEST', 'EAST');
    expect(() => e.ANYWHERE = 'ANYWHERE').to.throw();
  });

  it('should be able to validate enum keys', () => {
    let e = Enum('NORTH', 'SOUTH', 'WEST', 'EAST');
    expect(e.isValid('SOUTH')).to.be.true;
    expect(e.isValid('ANYWHERE')).to.be.false;
    expect(e.isValid()).to.be.false;
    expect(e.isValid(null)).to.be.false;
  })

});
