import Gettext from 'node-gettext';
import { LocalizationContexts } from './localization-contexts';
import log from './logging';

const SOURCE_LANGUAGE = 'en';

function setErrorHandler(catalogue: Gettext) {
  catalogue.on('error', (error) => {
    log.warn(`Gettext error: ${error}`);
  });
}

const gettextOptions = { sourceLocale: SOURCE_LANGUAGE };

declare class GettextWithAppContexts extends Gettext {
  pgettext(msgctxt: LocalizationContexts, msgid: string): string;
  npgettext(
    msgctxt: LocalizationContexts,
    msgid: string,
    msgidPlural: string,
    count: number,
  ): string;
}

export const messages = new Gettext(gettextOptions) as GettextWithAppContexts;
messages.setTextDomain('messages');
setErrorHandler(messages);

export const relayLocations = new Gettext(gettextOptions);
relayLocations.setTextDomain('relay-locations');
setErrorHandler(relayLocations);
