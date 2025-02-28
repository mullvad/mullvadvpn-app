/* eslint-disable react/jsx-no-bind */
import { Button } from '../../../lib/components';
import { useCustomComponentContext } from '../context';

function ResetButton() {
  const { values, setValues } = useCustomComponentContext();
  const { initialCount } = values;

  function handleClick() {
    setValues({
      count: initialCount,
    });
  }

  return (
    <div style={{ padding: '8px' }}>
      <Button variant="destructive" onClick={handleClick}>
        Reset count
      </Button>
    </div>
  );
}

export default ResetButton;
