/**
 * Polls `getDriveList` once a second while mounted (upstream's drive-list
 * poll thread) and reports the latest validated answer.
 */
import { useEffect, useRef, useState } from 'react'

import { call } from '@/api/imager'

import { type DriveList, emptyDriveList, parseDriveList } from './drives'

export const DRIVE_POLL_MS = 1000

/** Calls `onList` with every answer; `enabled` false stops polling. */
export function usePollDrives(onList: (list: DriveList) => void, enabled = true): void {
  const handler = useRef(onList)
  useEffect(() => {
    handler.current = onList
  })
  useEffect(() => {
    if (!enabled) return
    let live = true
    const poll = () => {
      call('getDriveList').then(
        (value) => {
          if (live) handler.current(parseDriveList(value))
        },
        (error: unknown) => {
          if (live) handler.current({ drives: [], error: String(error) })
        },
      )
    }
    poll()
    const timer = setInterval(poll, DRIVE_POLL_MS)
    return () => {
      live = false
      clearInterval(timer)
    }
  }, [enabled])
}

/** The current drive list, refreshed every second. */
export function useDriveList(onList?: (list: DriveList) => void): DriveList {
  const [list, setList] = useState<DriveList>(emptyDriveList)
  usePollDrives((next) => {
    setList(next)
    onList?.(next)
  })
  return list
}
