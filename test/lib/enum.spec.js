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

});
