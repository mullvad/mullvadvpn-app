import CountComponent from './custom-context-component';
import ReactContextComponent from './react-context-component/react-context';

export default function AppRouter() {
  return <CountComponent heading="Count with me" initialCount={0} />;
}
