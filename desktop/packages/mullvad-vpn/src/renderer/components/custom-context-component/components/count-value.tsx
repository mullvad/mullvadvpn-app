import { useCustomComponentContext } from '../context';

function CountValue() {
  const { values } = useCustomComponentContext();
  const { count } = values;

  return (
    <div style={{ padding: '8px' }}>
      Count value: <sub style={{ fontSize: '24px', fontWeight: 'bold' }}>{count}</sub>
    </div>
  );
}

export default CountValue;
