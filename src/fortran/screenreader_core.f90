module screenreader_core
  implicit none
  integer, parameter :: max_events = 16
  type :: accessible_node
    character(len=128) :: role = ''
    character(len=256) :: name = ''
    character(len=256) :: value = ''
    logical :: focused = .false.
    logical :: disabled = .false.
  end type
  type :: screenreader_event
    integer :: kind = 1
    type(accessible_node) :: node
  end type
  type :: event_queue
    type(screenreader_event) :: items(max_events)
    integer :: head = 1
    integer :: count = 0
  end type
contains
  function announce(node) result(text)
    type(accessible_node), intent(in) :: node
    character(len=768) :: text
    character(len=128) :: role
    text = ''
    if (.not. node%focused) return
    role = trim(node%role)
    if (len_trim(role) == 0) role = 'element'
    text = trim(role)
    if (len_trim(node%name) > 0) text = trim(text)//': '//trim(node%name)
    if (len_trim(node%value) > 0) text = trim(text)//', '//trim(node%value)
    if (node%disabled) text = trim(text)//', disabled'
  end function
  logical function queue_push(q, event)
    type(event_queue), intent(inout) :: q
    type(screenreader_event), intent(in) :: event
    integer :: index
    queue_push = .false.
    if (q%count >= max_events) return
    index = mod(q%head - 1 + q%count, max_events) + 1
    q%items(index) = event
    q%count = q%count + 1
    queue_push = .true.
  end function
  logical function queue_pop(q, event)
    type(event_queue), intent(inout) :: q
    type(screenreader_event), intent(out) :: event
    queue_pop = .false.
    if (q%count == 0) return
    event = q%items(q%head)
    q%head = mod(q%head, max_events) + 1
    q%count = q%count - 1
    queue_pop = .true.
  end function
end module
