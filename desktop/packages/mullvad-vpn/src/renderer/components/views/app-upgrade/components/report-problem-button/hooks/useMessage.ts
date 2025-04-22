import { messages } from '../../../../../../../shared/gettext';
import { useErrorMessage } from './useErrorMessage';

// TRANSLATORS: Instruction to the user shown pre-filled in a form where the user
// TRANSLATORS: is asked to add more details about a failed upgrade.
const POST_ERROR_MESSAGE_USER_INSTRUCTIONS = messages.pgettext(
  'app-upgrade-view',
  'Add more details here:',
);

export const useMessage = () => {
  const errorMessage = useErrorMessage();

  if (errorMessage) {
    const message = `${errorMessage}\n\n${POST_ERROR_MESSAGE_USER_INSTRUCTIONS}\n\n`;

    return message;
  }

  return null;
};
