# Accordion

Header uses `ListItem` component.

## Examples

```tsx
export function Accordion() {
  const [expanded, setExpanded] = React.useState(false);

  return (
    <Accordion expanded={expanded} onExpandedChange={setExpanded}>
      <Accordion.Header>
        <Accordion.Header.Trigger>
          <Accordion.Header.Item>
            <Accordion.Header.Item.Title>Title</Accordion.Header.Item.Title>
            <Accordion.Header.Item.ActionGroup>
              <Accordion.Header.Item.Chevron />
            </Accordion.Header.Item.ActionGroup>
          </Accordion.Header.Item>
        </Accordion.Header.Trigger>
      </Accordion.Header>

      <Accordion.Content>
        <div>Content</div>
      </Accordion.Content>
    </Accordion>
  );
}
```
