class Component extends DCLogic {
  renderVals() {
    return { theme: this.props.dunkel ? 'dunkel' : '' };
  }
}
