import * as React from 'react';
import styled from 'styled-components';

import { spacings } from '../tokens';
import { LabelTiny, TitleBig } from './common/text';

export const Container = styled.div({
  flex: 0,
  margin: `${spacings.spacing3} ${spacings.spacing5} ${spacings.spacing4}`,
});

export const ContentWrapper = styled.div({
  '&&:not(:first-child)': {
    paddingTop: '8px',
  },
});

export const HeaderTitle = styled(TitleBig)({
  wordWrap: 'break-word',
  hyphens: 'auto',
});

export const HeaderSubTitle = styled(LabelTiny).attrs({
  $color: 'white60',
})({});

interface ISettingsHeaderProps {
  children?: React.ReactNode;
  className?: string;
}

function SettingsHeader(props: ISettingsHeaderProps, forwardRef: React.Ref<HTMLDivElement>) {
  return (
    <Container ref={forwardRef} className={props.className}>
      {React.Children.map(props.children, (child) => {
        return React.isValidElement(child) ? <ContentWrapper>{child}</ContentWrapper> : undefined;
      })}
    </Container>
  );
}

export default React.forwardRef(SettingsHeader);
