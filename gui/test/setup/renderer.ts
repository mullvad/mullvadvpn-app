import Enzyme from 'enzyme';
import ReactSixteenAdapter from 'enzyme-adapter-react-16';
import chai from 'chai';
import spies from 'chai-spies';
import chaiAsPromised from 'chai-as-promised';

chai.use(spies);
chai.use(chaiAsPromised);

Enzyme.configure({
  adapter: new ReactSixteenAdapter(),
});
