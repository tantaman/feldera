// A create/update dialog for a Kafka input connector.
'use client'

import TabFooter from '$lib/components/connectors/dialogs/common/TabFooter'
import TabLabel from '$lib/components/connectors/dialogs/common/TabLabel'
import { connectorTypeToConfig, parseUrlSchema } from '$lib/functions/connectors'
import { PLACEHOLDER_VALUES } from '$lib/functions/placeholders'
import { useConnectorRequest } from '$lib/services/connectors/dialogs/SubmitHandler'
import { ConnectorType } from '$lib/types/connectors'
import ConnectorDialogProps from '$lib/types/connectors/ConnectorDialogProps'
import { useEffect, useState } from 'react'
import { FieldErrors } from 'react-hook-form'
import { FormContainer, TextFieldElement } from 'react-hook-form-mui'
import * as va from 'valibot'

import { valibotResolver } from '@hookform/resolvers/valibot'
import { Icon } from '@iconify/react'
import TabContext from '@mui/lab/TabContext'
import TabList from '@mui/lab/TabList'
import TabPanel from '@mui/lab/TabPanel'
import { Grid } from '@mui/material'
import Box from '@mui/material/Box'
import Dialog from '@mui/material/Dialog'
import DialogContent from '@mui/material/DialogContent'
import IconButton from '@mui/material/IconButton'
import Tab from '@mui/material/Tab'
import Typography from '@mui/material/Typography'

import TabInputFormatDetails from './common/TabInputFormatDetails'
import Transition from './common/Transition'

const schema = va.object({
  name: va.nonOptional(va.string()),
  description: va.optional(va.string(), ''),
  url: va.nonOptional(va.string()),
  format_name: va.nonOptional(va.enumType(['json', 'csv'])),
  json_update_format: va.optional(va.enumType(['raw', 'insert_delete']), 'raw'),
  json_array: va.nonOptional(va.boolean())
})

export type UrlSchema = va.Input<typeof schema>

export const UrlConnectorDialog = (props: ConnectorDialogProps) => {
  const [activeTab, setActiveTab] = useState<string>('detailsTab')
  const [curValues, setCurValues] = useState<UrlSchema | undefined>(undefined)

  // Initialize the form either with default or values from the passed in connector
  useEffect(() => {
    if (props.connector) {
      setCurValues(parseUrlSchema(props.connector))
    }
  }, [props.connector])

  const defaultValues: UrlSchema = {
    name: '',
    description: '',
    url: '',
    format_name: 'json',
    json_update_format: 'raw',
    json_array: false
  }

  const handleClose = () => {
    setActiveTab(tabList[0])
    props.setShow(false)
  }

  // Define what should happen when the form is submitted
  const prepareData = (data: UrlSchema) => ({
    name: data.name,
    description: data.description,
    config: {
      transport: {
        name: connectorTypeToConfig(ConnectorType.URL),
        config: {
          path: data.url
        }
      },
      format: {
        name: data.format_name,
        config:
          data.format_name === 'json'
            ? {
                update_format: data.json_update_format,
                array: data.json_array
              }
            : {}
      }
    }
  })

  const onSubmit = useConnectorRequest(props.connector, prepareData, props.onSuccess, handleClose)

  // If there is an error, switch to the earliest tab with an error
  const handleErrors = (errors: FieldErrors<UrlSchema>) => {
    if (!props.show) {
      return
    }
    if (errors?.name || errors?.description || errors?.url) {
      setActiveTab('detailsTab')
    } else if (errors?.format_name || errors?.json_array || errors?.json_update_format) {
      setActiveTab('formatTab')
    }
  }

  const tabList = ['detailsTab', 'formatTab']
  const tabFooter = (
    <TabFooter
      isUpdate={props.connector !== undefined}
      activeTab={activeTab}
      setActiveTab={setActiveTab}
      tabsArr={tabList}
    />
  )
  return (
    <Dialog
      fullWidth
      open={props.show}
      scroll='body'
      maxWidth='md'
      onClose={handleClose}
      TransitionComponent={Transition}
    >
      <FormContainer
        resolver={valibotResolver(schema)}
        values={curValues}
        defaultValues={defaultValues}
        onSuccess={onSubmit}
        onError={handleErrors}
      >
        <DialogContent
          sx={{
            pt: { xs: 8, sm: 12.5 },
            pr: { xs: 5, sm: 12 },
            pb: { xs: 5, sm: 9.5 },
            pl: { xs: 4, sm: 11 },
            position: 'relative'
          }}
        >
          <IconButton size='small' onClick={handleClose} sx={{ position: 'absolute', right: '1rem', top: '1rem' }}>
            <Icon icon='bx:x' />
          </IconButton>
          <Box sx={{ mb: 8, textAlign: 'center' }}>
            <Typography variant='h5' sx={{ mb: 3 }}>
              {props.connector === undefined ? 'New URL' : 'Update ' + props.connector.name}
            </Typography>
            {props.connector === undefined && <Typography variant='body2'>Provide the URL to a data source</Typography>}
          </Box>
          <Box sx={{ display: 'flex', flexWrap: { xs: 'wrap', md: 'nowrap' } }}>
            <TabContext value={activeTab}>
              <TabList
                orientation='vertical'
                onChange={(e, newValue: string) => setActiveTab(newValue)}
                sx={{
                  border: 0,
                  minWidth: 200,
                  '& .MuiTabs-indicator': { display: 'none' },
                  '& .MuiTabs-flexContainer': {
                    alignItems: 'flex-start',
                    '& .MuiTab-root': {
                      width: '100%',
                      alignItems: 'flex-start'
                    }
                  }
                }}
              >
                <Tab
                  disableRipple
                  value='detailsTab'
                  label={
                    <TabLabel
                      title='Source'
                      subtitle='Description'
                      active={activeTab === 'detailsTab'}
                      icon={<Icon icon='bx:file' />}
                    />
                  }
                />
                <Tab
                  disableRipple
                  value='formatTab'
                  label={
                    <TabLabel
                      title='Format'
                      active={activeTab === 'formatTab'}
                      subtitle='Data details'
                      icon={<Icon icon='lucide:file-json-2' />}
                    />
                  }
                />
              </TabList>
              <TabPanel
                value='detailsTab'
                sx={{ border: 0, boxShadow: 0, width: '100%', backgroundColor: 'transparent' }}
              >
                <Grid container spacing={4}>
                  <Grid item sm={4} xs={12}>
                    <TextFieldElement
                      name='name'
                      label='Datasource Name'
                      size='small'
                      fullWidth
                      placeholder={PLACEHOLDER_VALUES['connector_name']}
                      aria-describedby='validation-name'
                    />
                  </Grid>
                  <Grid item sm={8} xs={12}>
                    <TextFieldElement
                      name='description'
                      label='Description'
                      size='small'
                      fullWidth
                      placeholder={PLACEHOLDER_VALUES['connector_description']}
                      aria-describedby='validation-description'
                    />
                  </Grid>
                  <Grid item sm={12} xs={12}>
                    <TextFieldElement
                      name='url'
                      label='URL'
                      size='small'
                      fullWidth
                      placeholder='https://gist.githubusercontent.com/...'
                      aria-describedby='validation-description'
                    />
                  </Grid>
                </Grid>

                {tabFooter}
              </TabPanel>
              <TabPanel
                value='formatTab'
                sx={{ border: 0, boxShadow: 0, width: '100%', backgroundColor: 'transparent' }}
              >
                {/* @ts-ignore: TODO: This type mismatch seems like a bug in hook-form and/or resolvers */}
                <TabInputFormatDetails />
                {tabFooter}
              </TabPanel>
            </TabContext>
          </Box>
        </DialogContent>
      </FormContainer>
    </Dialog>
  )
}
