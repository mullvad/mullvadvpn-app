// @flow
import { MobileAppBridge } from 'NativeModules';

const collectProblemReport = (toRedact: string) => {
  return MobileAppBridge.collectProblemReport(toRedact);
};

const sendProblemReport = (email: string, message: string, savedReport: string) => {
  return MobileAppBridge.sendProblemReport(email, message, savedReport);
};

export { collectProblemReport, sendProblemReport };
