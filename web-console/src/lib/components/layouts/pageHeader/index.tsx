import { ReactNode } from 'react'

import { Box, Typography } from '@mui/material'

const PageHeader = (props: { title: ReactNode; subtitle?: ReactNode }) => {
  const { title, subtitle } = props

  return (
    <Box sx={{ mt: '-3rem', pl: { xs: '5rem', lg: '2rem' } }}>
      {typeof title === 'string' ? <Typography variant='h5'>{title}</Typography> : title}
      {<Typography variant='body2'>{subtitle}</Typography>}
    </Box>
  )
}

export default PageHeader
