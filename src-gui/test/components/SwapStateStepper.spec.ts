import { expect, test } from 'vitest'
import { render, screen } from '@testing-library/react'
import { userEvent } from '@testing-library/user-event'

test('Swap state stepper renders', async () => {
  render(
    <Link page="http://antfu.me">Anthony Fu</Link>,
  )

  const link = screen.getByText('Anthony Fu')

  expect(link).toHaveAccessibleName('Link is normal')

  await userEvent.hover(link)

  await expect.poll(() => link).toHaveAccessibleName('Link is hovered')

  await userEvent.unhover(link)

  await expect.poll(() => link).toHaveAccessibleName('Link is normal')
})
