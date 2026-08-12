# Wizard

Extension of `Carousel` component, used to display instructions or information over several slides.

## Example

```tsx
<Wizard>
  <Wizard.Slides>
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>Slide 1</Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text>
          Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
        </Wizard.Slides.Slide.Text>
      </Wizard.Slides.Slide.TextGroup>
    </Wizard.Slides.Slide>
    <Wizard.Slides.Slide>
      <Wizard.Slides.Slide.Icon icon="info-circle" />
      <Wizard.Slides.Slide.Title>Slide 2</Wizard.Slides.Slide.Title>
      <Wizard.Slides.Slide.TextGroup>
        <Wizard.Slides.Slide.Text>
          Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
        </Wizard.Slides.Slide.Text>
        <Wizard.Slides.Slide.Text>
          Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        </Wizard.Slides.Slide.Text>
      </Wizard.Slides.Slide.TextGroup>
    </Wizard.Slides.Slide>
  </Wizard.Slides>
  <Wizard.Controls>
    <Wizard.Controls.Indicators />
    <Wizard.Controls.ButtonGroup>
      <Wizard.Controls.NextButton />
      <Wizard.Controls.PrevButton />
      <Wizard.Controls.ExitButton />
    </Wizard.Controls.ButtonGroup>
  </Wizard.Controls>
</Wizard>
```
