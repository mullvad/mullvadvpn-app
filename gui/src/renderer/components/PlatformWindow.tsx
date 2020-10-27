import styled from 'styled-components';

const ARROW_WIDTH = 30;

export default styled.div({}, ({ arrowPosition }: { arrowPosition?: number }) => {
  let mask: string | undefined;

  if (process.platform === 'darwin') {
    const arrowPositionCss =
      arrowPosition !== undefined ? `${arrowPosition - ARROW_WIDTH * 0.5}px` : '50%';

    mask = [
      `url(../../assets/images/app-triangle.svg) ${arrowPositionCss} 0% no-repeat`,
      'url(../../assets/images/app-header-backdrop.svg) no-repeat',
    ].join(',');
  }

  return {
    position: 'relative',
    overflow: 'hidden',
    display: 'flex',
    flexDirection: 'column',
    flex: 1,
    mask,
  };
});
