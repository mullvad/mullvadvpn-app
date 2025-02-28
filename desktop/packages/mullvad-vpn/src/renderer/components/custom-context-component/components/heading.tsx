export type HeadingProps = {
  children: React.ReactNode;
};

function Heading({ children }: HeadingProps) {
  return <h1 style={{ fontSize: '32px', paddingBottom: '8px' }}>{children}</h1>;
}

export default Heading;
