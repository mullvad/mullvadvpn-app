import { JSDOM } from 'jsdom';
import Enzyme from 'enzyme';
import ReactSixteenAdapter from 'enzyme-adapter-react-16';

Enzyme.configure({
  adapter: new ReactSixteenAdapter(),
});

// @ts-ignore
const jsdom = new JSDOM('<!doctype html><html><body></body></html>');
