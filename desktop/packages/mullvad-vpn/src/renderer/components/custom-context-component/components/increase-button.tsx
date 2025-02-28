/* eslint-disable react/jsx-no-bind */
import { Button } from '../../../lib/components';
import { useCustomComponentContext } from '../context';

function IncreaseButton() {
  const { values, setValues } = useCustomComponentContext();
  const { count } = values;

  function handleClick() {
    setValues({
      count: count + 1,
    });
  }

  return (
    <div style={{ padding: '8px' }}>
      <Button variant="primary" onClick={handleClick}>
        Increase count
      </Button>
    </div>
  );
}

export default IncreaseButton;
