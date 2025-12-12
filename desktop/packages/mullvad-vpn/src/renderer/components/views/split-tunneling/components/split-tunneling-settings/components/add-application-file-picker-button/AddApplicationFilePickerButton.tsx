import { messages } from '../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../lib/components';
import { Container } from '../../../../../../../lib/components';
import { useAddWithFilePicker } from '../../hooks';

export function AddApplicationFilePickerButton() {
  const addWithFilePicker = useAddWithFilePicker();

  return (
    <Container indent="large">
      <Button onClick={addWithFilePicker}>
        <Button.Text>{messages.pgettext('split-tunneling-view', 'Find another app')}</Button.Text>
      </Button>
    </Container>
  );
}
