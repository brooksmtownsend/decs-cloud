import React, { Component } from 'react';
import {
} from 'reactstrap';


class ScoreEntry extends Component {
  constructor(props) {
    super(props);

    this.state = {
      display_name: this.props.score.player
    }
  }

  onUpdate = () => {
    this.setState({});
  }

  componentDidMount() {
    this.props.score.on('change', this.onUpdate);
    if (this.props.score.player !== "Nobody") {
      this.props.client.get(`decs.components.mainworld.${this.props.score.player}.transponder`).then(t => {
        this.setState({ display_name: t.display_name })
      })
    }
  }

  componentWillUnmount() {
    this.props.score.off('change', this.onUpdate);
  }
  render() {
    return (
      <tr>
        <td>{this.state.display_name}</td>
        <td>{this.props.score.amount}</td>
      </tr>
    )
  }
}

export default ScoreEntry;
