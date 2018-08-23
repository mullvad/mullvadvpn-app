import { JSDOM } from 'jsdom';
import * as Enzyme from 'enzyme';
import * as Adapter from 'enzyme-adapter-react-16';
import * as chai from 'chai';

Enzyme.configure({
  adapter: new Adapter(),
});

const jsdom = new JSDOM('<!doctype html><html><body></body></html>');
