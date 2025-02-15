// The properties passed to a create/update connector dialog.

import { ConnectorDescr } from '$lib/services/manager'
import { Dispatch, SetStateAction } from 'react'

interface ConnectorDialogProps {
  connector?: ConnectorDescr
  show: boolean
  setShow: Dispatch<SetStateAction<boolean>>
  onSuccess?: Dispatch<ConnectorDescr>
}

export default ConnectorDialogProps
