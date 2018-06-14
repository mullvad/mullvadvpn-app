const Enzyme = require('enzyme');
const Adapter = require('enzyme-adapter-react-16');
const chai = require('chai');
const spies = require('chai-spies');

chai.use(spies);

Enzyme.configure({
  adapter: new Adapter(),
});
