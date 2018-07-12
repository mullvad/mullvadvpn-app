const Enzyme = require('enzyme');
const Adapter = require('enzyme-adapter-react-16');
const chai = require('chai');
const spies = require('chai-spies');
const chaiAsPromised = require('chai-as-promised');

chai.use(spies);
chai.use(chaiAsPromised);

Enzyme.configure({
  adapter: new Adapter(),
});

global.expect = chai.expect;
global.spy = chai.spy;
