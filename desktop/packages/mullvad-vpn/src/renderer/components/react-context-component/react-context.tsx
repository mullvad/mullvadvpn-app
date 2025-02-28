/* eslint-disable react/jsx-no-bind */
import { createContext, useContext, useState } from 'react';

import { Button } from '../../lib/components';
import { Heading, Wrapper } from '../custom-context-component/components';

type Values = {
  count: number;
  setCount: (count: number) => void;
};

const CountContext = createContext<Values | undefined>(undefined);

const useCountContext = () => {
  const countContext = useContext(CountContext);

  if (!countContext) {
    throw new Error('Must use the CountContext Provider!');
  }

  return countContext;
};

function CurrentCount() {
  const { count } = useCountContext();

  return <div>Current count in a child {count}</div>;
}

function ButtonIncrease() {
  const { count, setCount } = useCountContext();

  function handleClick() {
    setCount(count + 1);
  }

  return (
    <Button variant="primary" onClick={handleClick}>
      Increase count
    </Button>
  );
}

type Props = {
  heading: string;
  initialCount: number;
};

export default function ReactContextComponent({ initialCount, heading }: Props) {
  const [count, setCount] = useState(initialCount);

  return (
    <CountContext.Provider value={{ count, setCount }}>
      <Wrapper>
        <Heading>{heading}</Heading>
        <CurrentCount />
        <ButtonIncrease />
      </Wrapper>
    </CountContext.Provider>
  );
}
