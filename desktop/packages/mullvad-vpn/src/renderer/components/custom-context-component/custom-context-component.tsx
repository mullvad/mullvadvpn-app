import { CountValue, Heading, IncreaseButton, ResetButton, Wrapper } from './components';
import { withComponentContextProvider } from './context';

export type CountContextComponentProps = {
  heading: string;
};

function CountComponent({ heading }: CountContextComponentProps) {
  return (
    <Wrapper>
      <Heading>{heading}</Heading>
      <CountValue />
      <IncreaseButton />
      <ResetButton />
    </Wrapper>
  );
}

const CountComponentWithContextProvider = withComponentContextProvider(CountComponent);

export default CountComponentWithContextProvider;
