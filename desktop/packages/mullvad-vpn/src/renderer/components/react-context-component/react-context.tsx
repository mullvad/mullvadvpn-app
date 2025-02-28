import { createContext, useContext } from 'react';

import { Heading, Wrapper } from '../custom-context-component/components';

type Values = {
  count: number;
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

type Props = {
  count: number;
  heading: string;
};

export default function ReactContextComponent({ count, heading }: Props) {
  return (
    <CountContext.Provider value={{ count }}>
      <Wrapper>
        <Heading>{heading}</Heading>
        <CurrentCount />
      </Wrapper>
    </CountContext.Provider>
  );
}
