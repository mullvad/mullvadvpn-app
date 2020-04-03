import log from 'electron-log';
import Enzyme from 'enzyme';
import ReactSixteenAdapter from 'enzyme-adapter-react-16';
import chai from 'chai';
import spies from 'chai-spies';
import chaiAsPromised from 'chai-as-promised';

log.transports.console.level = false;
log.transports.file.level = false;

chai.use(spies);
chai.use(chaiAsPromised);

Enzyme.configure({
  adapter: new ReactSixteenAdapter(),
});
