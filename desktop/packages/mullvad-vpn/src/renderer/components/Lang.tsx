import { PropsWithChildren } from 'react';
import styled from 'styled-components';

import { useSelector } from '../redux/store';

const StyledLang = styled.div({
  display: 'flex',
  flex: '1',
});

export default function Lang(props: PropsWithChildren) {
  const locale = useSelector((state) => state.userInterface.locale);
  return <StyledLang lang={locale}>{props.children}</StyledLang>;
}
