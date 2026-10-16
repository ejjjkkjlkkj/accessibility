program test_screenreader_core
  use screenreader_core
  implicit none
  type(accessible_node) :: node
  type(event_queue) :: queue
  type(screenreader_event) :: event, output_event
  integer :: i
  node%role = 'button'
  node%name = 'Confirm'
  node%focused = .true.
  if (trim(announce(node)) /= 'button: Confirm') error stop 'announce focused'
  node%focused = .false.
  if (len_trim(announce(node)) /= 0) error stop 'announce unfocused'
  event%kind = 1
  event%node = node
  do i = 1, max_events
    if (.not. queue_push(queue,event)) error stop 'queue push'
  end do
  if (queue_push(queue,event)) error stop 'queue overflow'
  if (.not. queue_pop(queue,output_event)) error stop 'queue pop'
  if (output_event%kind /= 1) error stop 'event kind'
  print *, 'PASS: Fortran portable core checks'
end program
