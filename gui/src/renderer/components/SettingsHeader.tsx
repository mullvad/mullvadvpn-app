import * as React from 'react';
import styled from 'styled-components';
import { bigText, smallText } from './common-styles';

export const Container = styled.div({
  flex: 0,
  padding: '2px 22px 20px',
});

export const ContentWrapper = styled.div({
  ':not(:first-child)': {
    paddingTop: '8px',
  },
});

export const HeaderTitle = styled.span(bigText);
export const HeaderSubTitle = styled.span(smallText);

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
