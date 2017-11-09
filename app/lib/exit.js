// @flow
import { remote } from 'electron';

module.exports = function () {
	remote.app.quit();
}
