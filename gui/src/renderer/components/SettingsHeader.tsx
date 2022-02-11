import * as React from 'react';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { hugeText, tinyText } from './common-styles';

export const Container = styled.div({
  flex: 0,
  padding: '2px 22px 20px',
});

export const ContentWrapper = styled.div({
  ':not(:first-child)': {
    paddingTop: '8px',
  },
});

export const HeaderTitle = styled.span(hugeText);
export const HeaderSubTitle = styled.span(tinyText, {
  color: colors.white60,
});

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
