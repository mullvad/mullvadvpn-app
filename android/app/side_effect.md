# Side Effects

## Types

- User action caused effect (user does an action, which then may or may not result in side effect)
- Delayed effect (user does an action, but may result in a delayed effect)
- External effect (External component causes a side effect to happen)

## Problems

- Consumer (view) might not be listening when effect happens
- Lifecycle? User leaves the app while when side effect happens.
- Delayed effect, e.g side effect happened, but user navigated away, should we care about it?

### Examples

- SelectLocation, user selects a location and it needs to be submitted.
  - Up for discussion with team, what is desired behaviour?

- Login, on success we should show a success state for 1 second, then navigate to ConnectScreen, we should not be able to miss event

- Splash, we need to produce a result and it must be consumed, we don't probably want to show an outdated result? E.g user opens app, swipes to other app, then opens it again.

- ConnectScreen, we might get a side effect as we navigate account (e.g OutOfTime), this event might be consumed but we need it to trigger again once we enter the ConnectScreen.
